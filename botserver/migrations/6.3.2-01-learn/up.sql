-- ============================================
-- Education / Learning Module
-- Version: 6.3.2
-- Issue: #625 - Digital Inclusion & Creator Portal
-- ============================================

CREATE TABLE IF NOT EXISTS learn_facts (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    user_id UUID,
    key TEXT NOT NULL,
    value JSONB NOT NULL,
    kind TEXT NOT NULL DEFAULT 'fact',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (bot_id, key)
);

CREATE INDEX IF NOT EXISTS idx_learn_facts_bot_user
    ON learn_facts (bot_id, user_id);

CREATE TABLE IF NOT EXISTS learn_lessons (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    user_id UUID,
    topic TEXT NOT NULL,
    kind TEXT NOT NULL,
    content JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_learn_lessons_bot_kind
    ON learn_lessons (bot_id, kind);

CREATE INDEX IF NOT EXISTS idx_learn_lessons_user_created
    ON learn_lessons (user_id, created_at DESC);
