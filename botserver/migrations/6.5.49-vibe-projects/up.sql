-- 6.5.49-vibe-projects
-- #743: dynamic project registry for the Vibe agent (replaces hardcoded
-- project/VM assumptions). Scoped like other SaaS tables (org/branch),
-- with the global default bot scope (nil UUIDs) as fallback.

CREATE TABLE IF NOT EXISTS vibe_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    name VARCHAR(255) NOT NULL,
    project_type VARCHAR(50) NOT NULL DEFAULT 'bot',
    repository VARCHAR(255) NOT NULL DEFAULT 'generalbots',
    framework VARCHAR(255),
    custom_domain VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    environment VARCHAR(50) NOT NULL DEFAULT 'development',
    payload JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_vibe_projects_name_branch ON vibe_projects(branch_id, name);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_branch ON vibe_projects(branch_id);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_status ON vibe_projects(status);
CREATE INDEX IF NOT EXISTS idx_vibe_projects_type ON vibe_projects(project_type);