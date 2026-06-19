-- Database Manager: query history and saved queries tables

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS database_query_history (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
        user_id UUID NOT NULL,
        query_text TEXT NOT NULL,
        is_mutation BOOLEAN NOT NULL DEFAULT FALSE,
        row_count BIGINT DEFAULT 0,
        duration_ms BIGINT DEFAULT 0,
        error_message TEXT,
        executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_db_query_history_bot_id ON database_query_history(bot_id);
    CREATE INDEX IF NOT EXISTS idx_db_query_history_user_id ON database_query_history(user_id);
    CREATE INDEX IF NOT EXISTS idx_db_query_history_executed_at ON database_query_history(executed_at DESC);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating database_query_history table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS database_saved_queries (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
        user_id UUID NOT NULL,
        name VARCHAR(255) NOT NULL,
        query_text TEXT NOT NULL,
        description TEXT,
        is_shared BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_db_saved_queries_bot_id ON database_saved_queries(bot_id);
    CREATE INDEX IF NOT EXISTS idx_db_saved_queries_user_id ON database_saved_queries(user_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating database_saved_queries table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE OR REPLACE FUNCTION update_db_saved_queries_updated_at()
    RETURNS TRIGGER AS $$
    BEGIN
        NEW.updated_at = NOW();
        RETURN NEW;
    END;
    $$ LANGUAGE plpgsql;

    DROP TRIGGER IF EXISTS trigger_db_saved_queries_updated_at ON database_saved_queries;
    CREATE TRIGGER trigger_db_saved_queries_updated_at
        BEFORE UPDATE ON database_saved_queries
        FOR EACH ROW
        EXECUTE FUNCTION update_db_saved_queries_updated_at();
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating trigger for database_saved_queries: %', SQLERRM;
END $$;
