-- Migration: 6.1.2_phase3_phase4
-- Description: Phase 3 and Phase 4 multi-agent features
-- Features:
--   - Episodic memory (conversation summaries)
--   - Knowledge graphs (entity relationships)
--   - Human-in-the-loop approvals
--   - LLM observability and cost tracking

-- ============================================
-- EPISODIC MEMORY TABLES
-- ============================================

-- Conversation episodes (summaries)
CREATE TABLE IF NOT EXISTS conversation_episodes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    summary TEXT NOT NULL,
    key_topics JSONB NOT NULL DEFAULT '[]',
    decisions JSONB NOT NULL DEFAULT '[]',
    action_items JSONB NOT NULL DEFAULT '[]',
    sentiment JSONB NOT NULL DEFAULT '{"score": 0, "label": "neutral", "confidence": 0.5}',
    resolution VARCHAR(50) NOT NULL DEFAULT 'unknown',
    message_count INTEGER NOT NULL DEFAULT 0,
    message_ids JSONB NOT NULL DEFAULT '[]',
    conversation_start TIMESTAMP WITH TIME ZONE NOT NULL,
    conversation_end TIMESTAMP WITH TIME ZONE NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for episodic memory
CREATE INDEX IF NOT EXISTS idx_episodes_user_id ON conversation_episodes(user_id);
CREATE INDEX IF NOT EXISTS idx_episodes_bot_id ON conversation_episodes(bot_id);
CREATE INDEX IF NOT EXISTS idx_episodes_session_id ON conversation_episodes(session_id);
CREATE INDEX IF NOT EXISTS idx_episodes_created_at ON conversation_episodes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_episodes_key_topics ON conversation_episodes USING GIN(key_topics);
CREATE INDEX IF NOT EXISTS idx_episodes_resolution ON conversation_episodes(resolution);

-- Full-text search on summaries
CREATE INDEX IF NOT EXISTS idx_episodes_summary_fts ON conversation_episodes
    USING GIN(to_tsvector('english', summary));

-- ============================================
-- KNOWLEDGE GRAPH TABLES
-- ============================================

-- Knowledge graph entities
CREATE TABLE IF NOT EXISTS kg_entities (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_name VARCHAR(500) NOT NULL,
    aliases JSONB NOT NULL DEFAULT '[]',
    properties JSONB NOT NULL DEFAULT '{}',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, entity_type, entity_name)
);

-- Knowledge graph relationships
CREATE TABLE IF NOT EXISTS kg_relationships (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    from_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    to_entity_id UUID NOT NULL REFERENCES kg_entities(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',
    confidence DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    bidirectional BOOLEAN NOT NULL DEFAULT false,
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, from_entity_id, to_entity_id, relationship_type)
);

-- Indexes for knowledge graph
CREATE INDEX IF NOT EXISTS idx_kg_entities_bot_id ON kg_entities(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name ON kg_entities(entity_name);
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_lower ON kg_entities(LOWER(entity_name));
CREATE INDEX IF NOT EXISTS idx_kg_entities_aliases ON kg_entities USING GIN(aliases);

CREATE INDEX IF NOT EXISTS idx_kg_relationships_bot_id ON kg_relationships(bot_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_from ON kg_relationships(from_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_to ON kg_relationships(to_entity_id);
CREATE INDEX IF NOT EXISTS idx_kg_relationships_type ON kg_relationships(relationship_type);

-- Full-text search on entity names
CREATE INDEX IF NOT EXISTS idx_kg_entities_name_fts ON kg_entities
    USING GIN(to_tsvector('english', entity_name));

-- ============================================
-- HUMAN-IN-THE-LOOP APPROVAL TABLES
-- ============================================

-- Approval requests
CREATE TABLE IF NOT EXISTS approval_requests (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    session_id UUID NOT NULL,
    initiated_by UUID NOT NULL,
    approval_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    channel VARCHAR(50) NOT NULL,
    recipient VARCHAR(500) NOT NULL,
    context JSONB NOT NULL DEFAULT '{}',
    message TEXT NOT NULL,
    timeout_seconds INTEGER NOT NULL DEFAULT 3600,
    default_action VARCHAR(50),
    current_level INTEGER NOT NULL DEFAULT 1,
    total_levels INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reminders_sent JSONB NOT NULL DEFAULT '[]',
    decision VARCHAR(50),
    decided_by VARCHAR(500),
    decided_at TIMESTAMP WITH TIME ZONE,
    comments TEXT
);

-- Approval chains
CREATE TABLE IF NOT EXISTS approval_chains (
    id UUID PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    bot_id UUID NOT NULL,
    levels JSONB NOT NULL DEFAULT '[]',
    stop_on_reject BOOLEAN NOT NULL DEFAULT true,
    require_all BOOLEAN NOT NULL DEFAULT false,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, name)
);

-- Approval audit log
CREATE TABLE IF NOT EXISTS approval_audit_log (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    actor VARCHAR(500) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(50),
    user_agent TEXT
);

-- Approval tokens (for secure links)
CREATE TABLE IF NOT EXISTS approval_tokens (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL REFERENCES approval_requests(id) ON DELETE CASCADE,
    token VARCHAR(100) NOT NULL UNIQUE,
    action VARCHAR(50) NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for approval tables
CREATE INDEX IF NOT EXISTS idx_approval_requests_bot_id ON approval_requests(bot_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_session_id ON approval_requests(session_id);
CREATE INDEX IF NOT EXISTS idx_approval_requests_status ON approval_requests(status);
CREATE INDEX IF NOT EXISTS idx_approval_requests_expires_at ON approval_requests(expires_at);
CREATE INDEX IF NOT EXISTS idx_approval_requests_pending ON approval_requests(status, expires_at)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_approval_audit_request_id ON approval_audit_log(request_id);
CREATE INDEX IF NOT EXISTS idx_approval_audit_timestamp ON approval_audit_log(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_approval_tokens_token ON approval_tokens(token);
CREATE INDEX IF NOT EXISTS idx_approval_tokens_request_id ON approval_tokens(request_id);

-- ============================================
-- LLM OBSERVABILITY TABLES
-- ============================================

-- LLM request metrics
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

-- Aggregated metrics (hourly rollup)
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

-- Budget tracking
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

-- Trace events
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

-- Indexes for observability tables
CREATE INDEX IF NOT EXISTS idx_llm_metrics_bot_id ON llm_metrics(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_session_id ON llm_metrics(session_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_timestamp ON llm_metrics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_model ON llm_metrics(model);

CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_bot_id ON llm_metrics_hourly(bot_id);
CREATE INDEX IF NOT EXISTS idx_llm_metrics_hourly_hour ON llm_metrics_hourly(hour DESC);

CREATE INDEX IF NOT EXISTS idx_llm_traces_trace_id ON llm_traces(trace_id);
CREATE INDEX IF NOT EXISTS idx_llm_traces_start_time ON llm_traces(start_time DESC);
CREATE INDEX IF NOT EXISTS idx_llm_traces_component ON llm_traces(component);

-- ============================================
-- WORKFLOW TABLES
-- ============================================

-- Workflow definitions
CREATE TABLE IF NOT EXISTS workflow_definitions (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    steps JSONB NOT NULL DEFAULT '[]',
    triggers JSONB NOT NULL DEFAULT '[]',
    error_handling JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(bot_id, name)
);

-- Workflow executions
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY,
    workflow_id UUID NOT NULL REFERENCES workflow_definitions(id) ON DELETE CASCADE,
    bot_id UUID NOT NULL,
    session_id UUID,
    initiated_by UUID,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    current_step INTEGER NOT NULL DEFAULT 0,
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB NOT NULL DEFAULT '{}',
    step_results JSONB NOT NULL DEFAULT '[]',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Workflow step executions
CREATE TABLE IF NOT EXISTS workflow_step_executions (
    id UUID PRIMARY KEY,
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    step_name VARCHAR(200) NOT NULL,
    step_index INTEGER NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB NOT NULL DEFAULT '{}',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_ms BIGINT
);

-- Indexes for workflow tables
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_bot_id ON workflow_definitions(bot_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id ON workflow_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_bot_id ON workflow_executions(bot_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX IF NOT EXISTS idx_workflow_step_executions_execution_id ON workflow_step_executions(execution_id);

-- ============================================
-- FUNCTIONS AND TRIGGERS
-- ============================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
DROP TRIGGER IF EXISTS update_kg_entities_updated_at ON kg_entities;
CREATE TRIGGER update_kg_entities_updated_at
    BEFORE UPDATE ON kg_entities
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_workflow_definitions_updated_at ON workflow_definitions;
CREATE TRIGGER update_workflow_definitions_updated_at
    BEFORE UPDATE ON workflow_definitions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_llm_budget_updated_at ON llm_budget;
CREATE TRIGGER update_llm_budget_updated_at
    BEFORE UPDATE ON llm_budget
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to aggregate hourly metrics
CREATE OR REPLACE FUNCTION aggregate_llm_metrics_hourly()
RETURNS void AS $$
DECLARE
    last_hour TIMESTAMP WITH TIME ZONE;
BEGIN
    last_hour := DATE_TRUNC('hour', NOW() - INTERVAL '1 hour');

    INSERT INTO llm_metrics_hourly (
        id, bot_id, hour, total_requests, successful_requests, failed_requests,
        cache_hits, cache_misses, total_input_tokens, total_output_tokens,
        total_tokens, total_cost, avg_latency_ms, p50_latency_ms, p95_latency_ms,
        p99_latency_ms, max_latency_ms, min_latency_ms, requests_by_model,
        tokens_by_model, cost_by_model
    )
    SELECT
        gen_random_uuid(),
        bot_id,
        last_hour,
        COUNT(*),
        COUNT(*) FILTER (WHERE success = true),
        COUNT(*) FILTER (WHERE success = false),
        COUNT(*) FILTER (WHERE cached = true),
        COUNT(*) FILTER (WHERE cached = false),
        SUM(input_tokens),
        SUM(output_tokens),
        SUM(total_tokens),
        SUM(estimated_cost),
        AVG(latency_ms),
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY latency_ms),
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms),
        PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms),
        MAX(latency_ms),
        MIN(latency_ms),
        jsonb_object_agg(model, model_count) FILTER (WHERE model IS NOT NULL),
        jsonb_object_agg(model, model_tokens) FILTER (WHERE model IS NOT NULL),
        jsonb_object_agg(model, model_cost) FILTER (WHERE model IS NOT NULL)
    FROM (
        SELECT
            bot_id, model, success, cached, input_tokens, output_tokens,
            total_tokens, estimated_cost, latency_ms,
            COUNT(*) OVER (PARTITION BY bot_id, model) as model_count,
            SUM(total_tokens) OVER (PARTITION BY bot_id, model) as model_tokens,
            SUM(estimated_cost) OVER (PARTITION BY bot_id, model) as model_cost
        FROM llm_metrics
        WHERE timestamp >= last_hour
        AND timestamp < last_hour + INTERVAL '1 hour'
    ) sub
    GROUP BY bot_id
    ON CONFLICT (bot_id, hour) DO UPDATE SET
        total_requests = EXCLUDED.total_requests,
        successful_requests = EXCLUDED.successful_requests,
        failed_requests = EXCLUDED.failed_requests,
        cache_hits = EXCLUDED.cache_hits,
        cache_misses = EXCLUDED.cache_misses,
        total_input_tokens = EXCLUDED.total_input_tokens,
        total_output_tokens = EXCLUDED.total_output_tokens,
        total_tokens = EXCLUDED.total_tokens,
        total_cost = EXCLUDED.total_cost,
        avg_latency_ms = EXCLUDED.avg_latency_ms,
        p50_latency_ms = EXCLUDED.p50_latency_ms,
        p95_latency_ms = EXCLUDED.p95_latency_ms,
        p99_latency_ms = EXCLUDED.p99_latency_ms,
        max_latency_ms = EXCLUDED.max_latency_ms,
        min_latency_ms = EXCLUDED.min_latency_ms,
        requests_by_model = EXCLUDED.requests_by_model,
        tokens_by_model = EXCLUDED.tokens_by_model,
        cost_by_model = EXCLUDED.cost_by_model;
END;
$$ LANGUAGE plpgsql;

-- Function to reset daily budget
CREATE OR REPLACE FUNCTION reset_daily_budgets()
RETURNS void AS $$
BEGIN
    UPDATE llm_budget
    SET daily_spend = 0,
        daily_reset_date = CURRENT_DATE,
        daily_alert_sent = false
    WHERE daily_reset_date < CURRENT_DATE;
END;
$$ LANGUAGE plpgsql;

-- Function to reset monthly budget
CREATE OR REPLACE FUNCTION reset_monthly_budgets()
RETURNS void AS $$
BEGIN
    UPDATE llm_budget
    SET monthly_spend = 0,
        monthly_reset_date = DATE_TRUNC('month', CURRENT_DATE)::DATE,
        monthly_alert_sent = false
    WHERE monthly_reset_date < DATE_TRUNC('month', CURRENT_DATE)::DATE;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- VIEWS
-- ============================================

-- View for recent episode summaries with user info
CREATE OR REPLACE VIEW v_recent_episodes AS
SELECT
    e.id,
    e.user_id,
    e.bot_id,
    e.session_id,
    e.summary,
    e.key_topics,
    e.sentiment,
    e.resolution,
    e.message_count,
    e.created_at,
    e.conversation_start,
    e.conversation_end
FROM conversation_episodes e
ORDER BY e.created_at DESC;

-- View for knowledge graph statistics
CREATE OR REPLACE VIEW v_kg_stats AS
SELECT
    bot_id,
    COUNT(DISTINCT id) as total_entities,
    COUNT(DISTINCT entity_type) as entity_types,
    (SELECT COUNT(*) FROM kg_relationships r WHERE r.bot_id = e.bot_id) as total_relationships
FROM kg_entities e
GROUP BY bot_id;

-- View for approval status summary
CREATE OR REPLACE VIEW v_approval_summary AS
SELECT
    bot_id,
    status,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (COALESCE(decided_at, NOW()) - created_at))) as avg_resolution_seconds
FROM approval_requests
GROUP BY bot_id, status;

-- View for LLM usage summary (last 24 hours)
CREATE OR REPLACE VIEW v_llm_usage_24h AS
SELECT
    bot_id,
    model,
    COUNT(*) as request_count,
    SUM(total_tokens) as total_tokens,
    SUM(estimated_cost) as total_cost,
    AVG(latency_ms) as avg_latency_ms,
    SUM(CASE WHEN cached THEN 1 ELSE 0 END)::FLOAT / COUNT(*) as cache_hit_rate,
    SUM(CASE WHEN success THEN 0 ELSE 1 END)::FLOAT / COUNT(*) as error_rate
FROM llm_metrics
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY bot_id, model;

-- ============================================
-- CLEANUP POLICIES (retention)
-- ============================================

-- Create a cleanup function for old data
CREATE OR REPLACE FUNCTION cleanup_old_observability_data(retention_days INTEGER DEFAULT 30)
RETURNS void AS $$
BEGIN
    -- Delete old LLM metrics (keep hourly aggregates longer)
    DELETE FROM llm_metrics WHERE timestamp < NOW() - (retention_days || ' days')::INTERVAL;

    -- Delete old traces
    DELETE FROM llm_traces WHERE start_time < NOW() - (retention_days || ' days')::INTERVAL;

    -- Delete old approval audit logs
    DELETE FROM approval_audit_log WHERE timestamp < NOW() - (retention_days * 3 || ' days')::INTERVAL;

    -- Delete expired approval tokens
    DELETE FROM approval_tokens WHERE expires_at < NOW() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;
