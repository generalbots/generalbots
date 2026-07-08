ALTER TABLE user_sessions ALTER COLUMN user_id TYPE UUID USING user_id::uuid;
ALTER TABLE user_sessions ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE user_sessions DROP COLUMN IF EXISTS data;
ALTER TABLE user_sessions DROP COLUMN IF EXISTS expires_at;
ALTER TABLE user_sessions DROP COLUMN IF EXISTS session_id;
ALTER TABLE user_sessions ADD CONSTRAINT user_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
