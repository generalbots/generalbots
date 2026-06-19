DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS handoff_contexts (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        organization_id UUID NOT NULL,
        session_id UUID NOT NULL,
        from_channel VARCHAR(50) NOT NULL,
        to_channel VARCHAR(50) NOT NULL,
        from_agent_id UUID,
        to_agent_id UUID,
        context_snapshot JSONB NOT NULL DEFAULT '{}',
        conversation_transcript JSONB NOT NULL DEFAULT '[]',
        customer_info JSONB NOT NULL DEFAULT '{}',
        priority VARCHAR(20) NOT NULL DEFAULT 'normal',
        status VARCHAR(30) NOT NULL DEFAULT 'pending',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        transferred_at TIMESTAMPTZ,
        completed_at TIMESTAMPTZ,
        notes TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_handoff_session ON handoff_contexts(session_id);
    CREATE INDEX IF NOT EXISTS idx_handoff_status ON handoff_contexts(status);
    CREATE INDEX IF NOT EXISTS idx_handoff_bot ON handoff_contexts(bot_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating handoff_contexts table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS conversation_analytics (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        organization_id UUID NOT NULL,
        session_id UUID NOT NULL,
        channel VARCHAR(50) NOT NULL,
        message_count INTEGER NOT NULL DEFAULT 0,
        user_message_count INTEGER NOT NULL DEFAULT 0,
        bot_message_count INTEGER NOT NULL DEFAULT 0,
        avg_response_time_ms INTEGER,
        total_tokens_used INTEGER NOT NULL DEFAULT 0,
        llm_cost_usd NUMERIC(10,6) NOT NULL DEFAULT 0,
        started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        ended_at TIMESTAMPTZ,
        duration_seconds INTEGER,
        resolved_by_human BOOLEAN NOT NULL DEFAULT false,
        metadata JSONB NOT NULL DEFAULT '{}'
    );

    CREATE INDEX IF NOT EXISTS idx_conv_analytics_bot ON conversation_analytics(bot_id);
    CREATE INDEX IF NOT EXISTS idx_conv_analytics_started ON conversation_analytics(started_at);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating conversation_analytics table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS conversation_ratings (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        organization_id UUID NOT NULL,
        session_id UUID NOT NULL,
        rating SMALLINT NOT NULL CHECK (rating BETWEEN 1 AND 5),
        comment TEXT,
        submitted_by UUID,
        submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        channel VARCHAR(50),
        tags JSONB NOT NULL DEFAULT '[]'
    );

    CREATE INDEX IF NOT EXISTS idx_conv_ratings_session ON conversation_ratings(session_id);
    CREATE INDEX IF NOT EXISTS idx_conv_ratings_bot ON conversation_ratings(bot_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating conversation_ratings table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS bot_metrics (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        organization_id UUID NOT NULL,
        metric_date DATE NOT NULL,
        conversations_total INTEGER NOT NULL DEFAULT 0,
        messages_total INTEGER NOT NULL DEFAULT 0,
        unique_users INTEGER NOT NULL DEFAULT 0,
        avg_rating NUMERIC(3,2),
        handoff_count INTEGER NOT NULL DEFAULT 0,
        resolution_rate NUMERIC(5,4),
        avg_session_seconds INTEGER,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        UNIQUE(bot_id, metric_date)
    );

    CREATE INDEX IF NOT EXISTS idx_bot_metrics_date ON bot_metrics(metric_date);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating bot_metrics table: %', SQLERRM;
END $$;
