-- GB6-16 — user preferences persist on the server and are read back into the
-- conversation prompt. Until now memory lived only in the browser's
-- localStorage, so the assistant could not use it in a fresh session.

CREATE TABLE IF NOT EXISTS user_memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    bot_id UUID,
    memory_key TEXT NOT NULL,
    memory_value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- A NULL bot_id means "all bots"; COALESCE is required because Postgres treats
-- NULLs as distinct in a plain unique constraint.
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_memories_unique
    ON user_memories (user_id, COALESCE(bot_id, '00000000-0000-0000-0000-000000000000'::uuid), memory_key);
CREATE INDEX IF NOT EXISTS idx_user_memories_user ON user_memories(user_id);
