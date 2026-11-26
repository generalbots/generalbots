-- Engine type and provider on each assistant row for per-engine usage grouping.
-- Additive: existing rows keep NULL; the usage summary classifies them by a
-- model-name fallback.
DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'message_history' AND column_name = 'llm_engine'
    ) THEN
        ALTER TABLE message_history ADD COLUMN llm_engine VARCHAR(20);
    END IF;
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'message_history' AND column_name = 'llm_provider'
    ) THEN
        ALTER TABLE message_history ADD COLUMN llm_provider VARCHAR(60);
    END IF;
END $$;
