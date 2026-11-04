-- #1681 — Agent Skills packages: keep the imported content + hash so the
-- capability block can advertise the skill and a changed hash can force
-- re-consent (same pinning idea as MCP UI resources).
ALTER TABLE bot_skills ADD COLUMN IF NOT EXISTS content TEXT;
ALTER TABLE bot_skills ADD COLUMN IF NOT EXISTS content_hash TEXT;
ALTER TABLE bot_skills ADD COLUMN IF NOT EXISTS version VARCHAR(64);
ALTER TABLE bot_skills ADD COLUMN IF NOT EXISTS enabled BOOLEAN NOT NULL DEFAULT TRUE;
