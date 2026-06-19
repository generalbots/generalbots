-- ============================================
-- Education / Learning Module
-- Version: 6.3.2
-- Issue: #625 - Digital Inclusion & Creator Portal
-- ============================================

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS learn_facts (
        id UUID PRIMARY KEY,
        bot_id UUID NOT NULL,
        user_id UUID,
        key TEXT NOT NULL,
        value JSONB NOT NULL,
        kind TEXT NOT NULL DEFAULT 'fact',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    -- Add UNIQUE constraint if not exists
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'learn_facts_bot_id_key'
    ) THEN
        ALTER TABLE learn_facts ADD CONSTRAINT learn_facts_bot_id_key UNIQUE (bot_id, key);
    END IF;

    CREATE INDEX IF NOT EXISTS idx_learn_facts_bot_user
        ON learn_facts (bot_id, user_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating learn_facts table: %', SQLERRM;
END $$;

DO $$
BEGIN
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
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating learn_lessons table: %', SQLERRM;
END $$;
