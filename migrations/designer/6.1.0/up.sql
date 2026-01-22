CREATE TABLE IF NOT EXISTS designer_dialogs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    bot_id TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_designer_dialogs_bot ON designer_dialogs(bot_id);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_active ON designer_dialogs(is_active);
CREATE INDEX IF NOT EXISTS idx_designer_dialogs_updated ON designer_dialogs(updated_at DESC);
