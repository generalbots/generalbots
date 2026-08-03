CREATE TABLE IF NOT EXISTS bot_skills (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    skill_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    installed_by UUID,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (bot_id, skill_id)
);
