ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS session_id VARCHAR;
ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS data JSONB;
ALTER TABLE user_sessions ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ;

DO $$ BEGIN
  ALTER TABLE user_sessions DROP CONSTRAINT IF EXISTS user_sessions_user_id_fkey;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;

ALTER TABLE user_sessions ALTER COLUMN user_id TYPE VARCHAR;
ALTER TABLE user_sessions ALTER COLUMN user_id SET NOT NULL;
