-- ============================================================================
-- PROJECTS TABLE (Issue #526)
-- ============================================================================
-- Tracks ALM projects (bots, apps, sites) with their deployment configuration
-- and status. Replaces the previous projects tracking in memory with a
-- persistent registry.

CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    project_type VARCHAR(50) NOT NULL,
    deploy_target VARCHAR(50) NOT NULL DEFAULT 'none',
    repo_url TEXT,
    deploy_url TEXT,
    container_name VARCHAR(255),
    custom_domain VARCHAR(255),
    environment VARCHAR(50) NOT NULL DEFAULT 'development',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    framework VARCHAR(100),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_projects_org_name UNIQUE (org, name)
);

CREATE INDEX IF NOT EXISTS idx_projects_org ON projects(org);
CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);
CREATE INDEX IF NOT EXISTS idx_projects_project_type ON projects(project_type);
CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at DESC);
