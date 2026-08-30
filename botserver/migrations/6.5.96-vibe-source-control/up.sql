-- 6.5.96-vibe-source-control
-- Per-project source-control mode for Vibe projects: 'native' (workspace
-- only, VM syncs from the workspace) or 'git' (Forgejo-backed, VM syncs from
-- the Forgejo repo, Deploy creates per-deploy branches and promotes
-- dev→prod by runtime).
ALTER TABLE vibe_projects ADD COLUMN IF NOT EXISTS source_control VARCHAR(50) NOT NULL DEFAULT 'native';
CREATE INDEX IF NOT EXISTS idx_vibe_projects_source_control ON vibe_projects(source_control);
